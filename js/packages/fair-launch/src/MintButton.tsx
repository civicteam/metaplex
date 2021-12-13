import styled from 'styled-components';
import Button from '@material-ui/core/Button';
import { CandyMachineAccount } from './candy-machine';
import { FairLaunchAccount } from './fair-launch';
import { CircularProgress } from '@material-ui/core';
import { GatewayProvider, GatewayStatus, useGateway, WalletAdapter } from '@civic/solana-gateway-react';
import { useEffect, useState } from 'react';
import {WalletContextState} from '@solana/wallet-adapter-react';

export const CTAButton = styled(Button)`
  width: 100%;
  height: 60px;
  margin-top: 10px;
  margin-bottom: 5px;
  background: linear-gradient(180deg, #604ae5 0%, #813eee 100%);
  color: white;
  font-size: 16px;
  font-weight: bold;
`; // add your styles here

enum GatewayRetrieveState {
  NotGateway,
  NotGettingGateway,
  InitialGetGateway,
  GettingGateway,
}

function isGateway(candyMachine: CandyMachineAccount | undefined, wallet: WalletContextState): boolean{
  return !!(candyMachine?.state?.isActive &&
    candyMachine.state.gatekeeper &&
    wallet.publicKey &&
    wallet.signTransaction);
}

export const MintButtonContext = ({
  wallet,
  onMint,
  candyMachine,
  fairLaunch,
  isMinting,
  fairLaunchBalance,
}: {
  wallet: WalletContextState;
  onMint: () => Promise<void>;
  candyMachine: CandyMachineAccount | undefined;
  fairLaunch?: FairLaunchAccount | undefined;
  isMinting: boolean;
  fairLaunchBalance: number;
}) => {
  const [gatewayRetrieveState, setGatewayRetrieveState] = useState<GatewayRetrieveState>(
    isGateway(candyMachine, wallet) ?
      GatewayRetrieveState.NotGettingGateway :
      GatewayRetrieveState.NotGateway
  );

  useEffect(() => {
    if (!isGateway(candyMachine, wallet)){
      setGatewayRetrieveState(GatewayRetrieveState.NotGateway);
    } else if (gatewayRetrieveState === GatewayRetrieveState.NotGateway) {
      setGatewayRetrieveState(GatewayRetrieveState.NotGettingGateway);
    }
  }, [candyMachine, wallet, gatewayRetrieveState, setGatewayRetrieveState])

  const button = <MintButton
    gatewayRetrieveState={gatewayRetrieveState}
    setGatewayRetrieveState={setGatewayRetrieveState}
    onMint={() => {
      if (gatewayRetrieveState !== GatewayRetrieveState.NotGateway){
        setGatewayRetrieveState(GatewayRetrieveState.NotGettingGateway);
      }
      return onMint();
    }}
    candyMachine={candyMachine}
    fairLaunch={fairLaunch}
    isMinting={isMinting}
    fairLaunchBalance={fairLaunchBalance}
  />;

  if (
    gatewayRetrieveState === GatewayRetrieveState.NotGateway ||
    gatewayRetrieveState === GatewayRetrieveState.NotGettingGateway
  ){
    return button;
  } else {
    return <GatewayProvider
      wallet={{
        publicKey: wallet.publicKey!,
        signTransaction: wallet.signTransaction!,
      }}
      gatekeeperNetwork={candyMachine?.state?.gatekeeper?.gatekeeperNetwork}
    >
      {button}
    </GatewayProvider>
  }
};

const MintButton = ({
  gatewayRetrieveState,
  setGatewayRetrieveState,
  onMint,
  candyMachine,
  fairLaunch,
  isMinting,
  fairLaunchBalance,
}: {
  gatewayRetrieveState: GatewayRetrieveState;
  setGatewayRetrieveState: (val: GatewayRetrieveState) => void;
  onMint: () => Promise<void>;
  candyMachine: CandyMachineAccount | undefined;
  fairLaunch?: FairLaunchAccount | undefined;
  isMinting: boolean;
  fairLaunchBalance: number;
}) => {
  const { requestGatewayToken, gatewayStatus } = useGateway();
  const [clicked, setClicked] = useState(false);

  useEffect(() => {
    if (gatewayStatus === GatewayStatus.ACTIVE && clicked) {
      console.log('Minting');
      onMint();
      setClicked(false);
    }
  }, [gatewayStatus, clicked, setClicked]);
  useEffect(() => {
    if (gatewayRetrieveState === GatewayRetrieveState.InitialGetGateway) {
      setGatewayRetrieveState(GatewayRetrieveState.GettingGateway);
      requestGatewayToken();
    }
  }, [gatewayRetrieveState, requestGatewayToken, setGatewayRetrieveState])
  return (
    <CTAButton
      disabled={
        candyMachine?.state.isSoldOut ||
        isMinting ||
        !candyMachine?.state.isActive ||
        (fairLaunch?.ticket?.data?.state.punched && fairLaunchBalance === 0)
      }
      onClick={async () => {
        setClicked(true);
        switch (gatewayRetrieveState){
          case GatewayRetrieveState.NotGateway:
            await onMint();
            setClicked(false);
            break;
          case GatewayRetrieveState.NotGettingGateway:
          case GatewayRetrieveState.InitialGetGateway:
            setGatewayRetrieveState(GatewayRetrieveState.InitialGetGateway);
            break;
          case GatewayRetrieveState.GettingGateway:
            if (gatewayStatus === GatewayStatus.ACTIVE) {
              setClicked(true);
            } else {
              await requestGatewayToken();
            }
            break;
        }
      }}
      variant="contained"
    >
      {fairLaunch?.ticket?.data?.state.punched && fairLaunchBalance === 0 ? (
        'MINTED'
      ) : candyMachine?.state.isSoldOut ? (
        'SOLD OUT'
      ) : isMinting ? (
        <CircularProgress />
      ) : (
        'MINT'
      )}
    </CTAButton>
  );
}
