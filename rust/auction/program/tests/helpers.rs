use std::str::FromStr;
use solana_program::{hash::Hash, program_pack::Pack, pubkey::Pubkey, system_instruction, system_program};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport::TransportError,
    sysvar
};
use solana_sdk::instruction::{AccountMeta, Instruction};
use metaplex_auction::{
    instruction,
    processor::{
        CancelBidArgs, ClaimBidArgs, CreateAuctionArgs, CreateAuctionArgsV2, EndAuctionArgs,
        PlaceBidArgs, PriceFloor, StartAuctionArgs, WinnerLimit,
    },
};

fn string_to_array(value: &str) -> Result<[u8; 32], TransportError> {
    if value.len() > 32 {
        return Err(TransportError::Custom("String too long".to_string()));
    }
    let mut result: [u8; 32] = Default::default();
    &result[0..value.len()].copy_from_slice(value.as_bytes());
    Ok(result)
}

pub async fn get_account(banks_client: &mut BanksClient, pubkey: &Pubkey) -> Account {
    banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn create_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
) -> Result<(Keypair, Keypair), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);
    let pool_mint = Keypair::new();
    let manager = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &pool_mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &pool_mint.pubkey(),
                &manager.pubkey(),
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, &pool_mint], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok((pool_mint, manager))
}

pub async fn create_token_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    account: &Keypair,
    pool_mint: &Pubkey,
    manager: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(spl_token::state::Account::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                pool_mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, account], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn mint_tokens(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    mint: &Pubkey,
    account: &Pubkey,
    mint_authority: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[payer, mint_authority],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn get_token_balance(banks_client: &mut BanksClient, token: &Pubkey) -> u64 {
    let token_account = banks_client.get_account(*token).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub async fn get_token_supply(banks_client: &mut BanksClient, mint: &Pubkey) -> u64 {
    let mint_account = banks_client.get_account(*mint).await.unwrap().unwrap();
    let account_info =
        spl_token::state::Mint::unpack_from_slice(mint_account.data.as_slice()).unwrap();
    account_info.supply
}

pub async fn create_store(
    banks_client: &mut BanksClient,
    program_id: &Pubkey,
    metaplex_program_id: &Pubkey,
    payer: &Keypair,
    recent_blockhash: &Hash,
) -> Result<Pubkey, TransportError> {
    let (store, _)= Pubkey::find_program_address(
        &[
            br"metaplex",
            &metaplex_program_id.to_bytes(),
            &payer.pubkey().to_bytes(),
        ],
        metaplex_program_id,
    );

    let metaplex_token_vault_id = Keypair::new().pubkey();
    let metaplex_token_metadata_id = Keypair::new().pubkey();

    let data = [08,00,00];  // add the data manually to avoid importing metaplex
    let keys = vec!(
        AccountMeta::new(store, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
        AccountMeta::new_readonly(payer.pubkey(), true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(metaplex_token_vault_id, false),
        AccountMeta::new_readonly(metaplex_token_metadata_id, false),
        AccountMeta::new_readonly(*program_id, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    );

    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            *metaplex_program_id,
            &data,
            keys,
        )],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash
    );

    banks_client.process_transaction(transaction).await?;
    Ok(store)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_auction(
    banks_client: &mut BanksClient,
    program_id: &Pubkey,
    payer: &Keypair,
    recent_blockhash: &Hash,
    resource: &Pubkey,
    gatekeeper_network: &Pubkey,
    mint_keypair: &Pubkey,
    max_winners: usize,
    name: &str,
    instant_sale_price: Option<u64>,
    price_floor: PriceFloor,
    gap_tick_size_percentage: Option<u8>,
    tick_size: Option<u64>,
    store: &Pubkey,
) -> Result<(), TransportError> {
    let transaction: Transaction;
    if instant_sale_price.is_some() {
        transaction = Transaction::new_signed_with_payer(
            &[instruction::create_auction_instruction_v2(
                *program_id,
                payer.pubkey(),
                *store,
                CreateAuctionArgsV2 {
                    authority: payer.pubkey(),
                    end_auction_at: None,
                    end_auction_gap: None,
                    resource: *resource,
                    gatekeeper_network: Some(*gatekeeper_network),
                    token_mint: *mint_keypair,
                    winners: WinnerLimit::Capped(max_winners),
                    price_floor,
                    gap_tick_size_percentage,
                    tick_size,
                    name: string_to_array(name).ok(),
                    instant_sale_price,
                },
            )],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        );
    } else {
        transaction = Transaction::new_signed_with_payer(
            &[instruction::create_auction_instruction(
                *program_id,
                payer.pubkey(),
                CreateAuctionArgs {
                    authority: payer.pubkey(),
                    end_auction_at: None,
                    end_auction_gap: None,
                    resource: *resource,
                    gatekeeper_network: Some(*gatekeeper_network),
                    token_mint: *mint_keypair,
                    winners: WinnerLimit::Capped(max_winners),
                    price_floor,
                    gap_tick_size_percentage,
                    tick_size,
                },
            )],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        );
    }
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn end_auction(
    banks_client: &mut BanksClient,
    program_id: &Pubkey,
    recent_blockhash: &Hash,
    payer: &Keypair,
    resource: &Pubkey,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::end_auction_instruction(
            *program_id,
            payer.pubkey(),
            EndAuctionArgs {
                resource: *resource,
                reveal: None,
            },
        )],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn start_auction(
    banks_client: &mut BanksClient,
    program_id: &Pubkey,
    recent_blockhash: &Hash,
    payer: &Keypair,
    resource: &Pubkey,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::start_auction_instruction(
            *program_id,
            payer.pubkey(),
            StartAuctionArgs {
                resource: *resource,
            },
        )],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn place_bid(
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    program_id: &Pubkey,
    payer: &Keypair,
    bidder: &Keypair,
    bidder_spl_account: &Keypair,
    bidder_gateway_token: &Pubkey,
    transfer_authority: &Keypair,
    resource: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::place_bid_instruction(
            *program_id,
            bidder.pubkey(),             // Wallet used to identify bidder
            *bidder_gateway_token,
            bidder.pubkey(), // SPL token account (source) using same account here for ease of testing
            bidder_spl_account.pubkey(), // SPL Token Account (Destination)
            *mint,           // Token Mint
            transfer_authority.pubkey(), // Approved to Move Tokens
            payer.pubkey(),  // Pays for Transactions
            PlaceBidArgs {
                amount,
                resource: *resource,
            },
        )],
        Some(&payer.pubkey()),
        &[bidder, transfer_authority, payer],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn cancel_bid(
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    program_id: &Pubkey,
    payer: &Keypair,
    bidder: &Keypair,
    bidder_spl_account: &Keypair,
    resource: &Pubkey,
    mint: &Pubkey,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::cancel_bid_instruction(
            *program_id,
            bidder.pubkey(),
            bidder.pubkey(),
            bidder_spl_account.pubkey(),
            *mint,
            CancelBidArgs {
                resource: *resource,
            },
        )],
        Some(&payer.pubkey()),
        &[bidder, payer],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn approve(
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    payer: &Keypair,
    transfer_authority: &Pubkey,
    spl_wallet: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[spl_token::instruction::approve(
            &spl_token::id(),
            &spl_wallet.pubkey(),
            transfer_authority,
            &payer.pubkey(),
            &[&payer.pubkey()],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn claim_bid(
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    program_id: &Pubkey,
    payer: &Keypair,
    authority: &Keypair,
    bidder: &Keypair,
    bidder_spl_account: &Keypair,
    seller: &Pubkey,
    resource: &Pubkey,
    mint: &Pubkey,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::claim_bid_instruction(
            *program_id,
            *seller,
            authority.pubkey(),
            bidder.pubkey(),
            bidder_spl_account.pubkey(),
            *mint,
            None,
            ClaimBidArgs {
                resource: *resource,
            },
        )],
        Some(&payer.pubkey()),
        &[payer, authority],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}
