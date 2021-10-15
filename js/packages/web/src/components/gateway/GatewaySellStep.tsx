import React, {useEffect} from "react";
import {Button, Row} from "antd";
import {GatekeeperNetworkSelection} from "../../contexts/gatekeeperNetwork";
import {AuctionState} from "../../views/auctionCreate";
import {omit} from "lodash";
import {getGatekeeperNetworkByKey} from "./GatewayRulesetSelector";
import {GatewayDescriptionText} from "./GatewayDescriptionText";
import {GatewayRulesetComponent} from "./GatewayRulesetComponent";
import {useMeta} from "@oyster/common";

export const GatewaySellStep:React.FC<{
  attributes: AuctionState;
  setAttributes: (attr: AuctionState) => void;
  confirm: () => void;
}> = ({
        attributes,
        setAttributes,
        confirm,
      }) => {

  const { store } = useMeta();

  const selectGatekeeperNetwork = (gkn?:GatekeeperNetworkSelection) => {
    if (!gkn) {
      // remove the gatekeeper network from the auction properties
      setAttributes(omit(attributes, 'gatekeeperNetwork'))
    } else {
      // set the selected gatekeeper network on the auction properties
      setAttributes({
        ...attributes,
        gatekeeperNetwork: gkn
      })
    }
  };

  // if the store has a proscribed gatekeeper network, add it to the auction
  useEffect(() => {
    if (store) {
      if (!store.info.gatekeeperNetwork) {
        selectGatekeeperNetwork(undefined);
      } else {
        selectGatekeeperNetwork(getGatekeeperNetworkByKey(store.info.gatekeeperNetwork))
      }
    }
  },[store])

  return (
    <>
      <GatewayDescriptionText/>
      {store?.info.gatekeeperNetwork ? <Row>
          <p>This store has been set up to use
            {getGatekeeperNetworkByKey(store.info.gatekeeperNetwork)?.name}
            for all auctions. For more details, see <a href='#'>here</a>.</p>
        </Row>
        :
        <GatewayRulesetComponent setGatekeeperNetwork={selectGatekeeperNetwork}/>
      }
      <Row>
        <Button
          type="primary"
          size="large"
          onClick={() => {
            confirm();
          }}
          className="action-btn"
        >
          Continue to review
        </Button>
      </Row>
    </>
  );
};
