import styled from 'styled-components';
import Button from '@material-ui/core/Button';
import { CandyMachineAccount } from './candy-machine';
import { CircularProgress } from '@material-ui/core';
import { GatewayStatus, useGateway } from '@civic/solana-gateway-react';
import { useEffect, useState } from 'react';

export const CTAButton = styled(Button)`
  width: 100%;
  height: 60px;
  margin-top: 10px;
  margin-bottom: 5px;
  background: linear-gradient(180deg, #604ae5 0%, #813eee 100%);
  color: white;
  font-size: 16px;
  font-weight: bold;
`; // add your own styles here

export const MintButton = ({
  onMint,
  candyMachine,
  isMinting,
}: {
  onMint: () => Promise<void>;
  candyMachine?: CandyMachineAccount;
  isMinting: boolean;
}) => {
  const { requestGatewayToken, gatewayStatus } = useGateway();
  const [clicked, setClicked] = useState(false);
  const [showEncore, setShowEncore] = useState(false);

  useEffect(() => {
    if (gatewayStatus === GatewayStatus.ACTIVE && clicked) {
      onMint();
      setClicked(false);
    }
  }, [gatewayStatus, clicked, setClicked, onMint]);

  const getMintButtonContent = () => {
    if (candyMachine?.state.isSoldOut) {
      return 'SOLD OUT';
    } else if (showEncore) {
      return 'Click when competed Encore Captcha';
    } else if (isMinting) {
      return <CircularProgress />;
    } else if (candyMachine?.state.isPresale) {
      return 'PRESALE MINT';
    }

    return 'MINT';
  };

  return (
    <>
      <CTAButton
        disabled={
          candyMachine?.state.isSoldOut ||
          isMinting ||
          !candyMachine?.state.isActive
        }
        onClick={async () => {
          setClicked(true);
          if (candyMachine?.state.isActive && candyMachine?.state.gatekeeper) {
            if (gatewayStatus === GatewayStatus.ACTIVE) {
              setClicked(true);
            } else {
              if (showEncore){
                setShowEncore(false);
                await requestGatewayToken();
              }
              else if (
                candyMachine.state.gatekeeper.gatekeeperNetwork.toBase58() ==
                'ign2PJfwxvYxAZpMdXgLdY4VLCnChPZWjtTeQwQfQdc'
              ) {
                setShowEncore(true);
                window.open('https://www.encore.fans/verify-hooman', '_blank')
              }
              else {
                await requestGatewayToken();
              }
            }
          } else {
            await onMint();
            setClicked(false);
          }
        }}
        variant="contained"
      >
        {getMintButtonContent()}
      </CTAButton>
    </>
  );
};
