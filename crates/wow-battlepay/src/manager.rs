use wow_packet::ServerPacket;
use wow_packet::packets::battlepay::{
    BattlePayAckFailedPacket, ConfirmPurchase, DeliveryEnded, DeliveryStarted,
    DistributionListResponse, DistributionObject, DistributionUpdate, ProductListResponse,
    PurchaseListResponse, PurchaseUpdate, StartPurchaseResponse,
};

/// Minimal BattlePay session facade used by the BattlePay crate.
pub trait BattlepaySession {
    fn send_packet<P: ServerPacket>(&self, packet: &P);
}

/// Send a minimal empty product list response.
pub fn send_empty_product_list(session: &impl BattlepaySession) {
    let resp = ProductListResponse {
        result: 0,
        currency_id: 0,
        product_infos: Vec::new(),
        products: Vec::new(),
        product_groups: Vec::new(),
        shops: Vec::new(),
    };

    session.send_packet(&resp);
}

/// Send a minimal empty purchase list response.
pub fn send_empty_purchase_list(session: &impl BattlepaySession) {
    let resp = PurchaseListResponse {
        result: 0,
        purchase: Vec::new(),
    };

    session.send_packet(&resp);
}

/// Send an empty distribution list response.
pub fn send_empty_distribution_list(session: &impl BattlepaySession) {
    let resp = DistributionListResponse {
        result: 0,
        distribution_object: Vec::new(),
    };

    session.send_packet(&resp);
}

/// Send a purchase list response.
pub fn send_purchase_list_response(
    session: &impl BattlepaySession,
    result: u32,
    purchases: Vec<wow_packet::packets::battlepay::Purchase>,
) {
    let resp = PurchaseListResponse {
        result,
        purchase: purchases,
    };

    session.send_packet(&resp);
}

/// Send a product list response.
pub fn send_product_list_response(
    session: &impl BattlepaySession,
    currency_id: u32,
    product_infos: Vec<wow_packet::packets::battlepay::ProductInfo>,
    products: Vec<wow_packet::packets::battlepay::Product>,
    product_groups: Vec<wow_packet::packets::battlepay::Group>,
    shops: Vec<wow_packet::packets::battlepay::Shop>,
) {
    let resp = ProductListResponse {
        result: 0,
        currency_id,
        product_infos,
        products,
        product_groups,
        shops,
    };

    session.send_packet(&resp);
}

/// Send a start purchase response.
pub fn send_start_purchase_response(
    session: &impl BattlepaySession,
    purchase_id: u64,
    client_token: u32,
    purchase_result: u32,
) {
    let resp = StartPurchaseResponse {
        purchase_id,
        client_token,
        purchase_result,
    };

    session.send_packet(&resp);
}

/// Send a purchase update notification.
pub fn send_purchase_update(
    session: &impl BattlepaySession,
    purchases: Vec<wow_packet::packets::battlepay::Purchase>,
) {
    let resp = PurchaseUpdate { purchase: purchases };
    session.send_packet(&resp);
}

/// Send a confirm purchase packet.
pub fn send_confirm_purchase(session: &impl BattlepaySession, purchase_id: u64, server_token: u32) {
    let resp = ConfirmPurchase {
        purchase_id,
        server_token,
    };
    session.send_packet(&resp);
}

/// Send a BattlePay ack failed packet.
pub fn send_ack_failed(
    session: &impl BattlepaySession,
    purchase_id: u64,
    purchase_result: u32,
    client_token: u32,
) {
    let resp = BattlePayAckFailedPacket {
        purchase_id,
        purchase_result,
        client_token,
    };
    session.send_packet(&resp);
}

/// Send a delivery started packet.
pub fn send_delivery_started(session: &impl BattlepaySession, distribution_id: u64) {
    let resp = DeliveryStarted { distribution_id };
    session.send_packet(&resp);
}

/// Send a delivery ended packet.
pub fn send_delivery_ended(
    session: &impl BattlepaySession,
    distribution_id: u64,
    item: Vec<Vec<u8>>,
) {
    let resp = DeliveryEnded { distribution_id, item };
    session.send_packet(&resp);
}

/// Send a distribution update packet.
pub fn send_distribution_update(
    session: &impl BattlepaySession,
    distribution_object: DistributionObject,
) {
    let resp = DistributionUpdate { distribution_object };
    session.send_packet(&resp);
}

/// Send a distribution list response.
pub fn send_distribution_list_response(
    session: &impl BattlepaySession,
    result: u32,
    distribution_objects: Vec<DistributionObject>,
) {
    let resp = DistributionListResponse {
        result,
        distribution_object: distribution_objects,
    };
    session.send_packet(&resp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use wow_packet::packets::battlepay::{ProductInfo, Product, Group, Shop, Purchase};
    use wow_constants::ServerOpcodes;
    use wow_packet::WorldPacket;

    struct DummySession {
        sent: RefCell<Vec<Vec<u8>>>,
    }

    impl DummySession {
        fn new() -> Self {
            Self { sent: RefCell::new(Vec::new()) }
        }

        fn last_opcode(&self) -> Option<u16> {
            self.sent.borrow().last().map(|packet| {
                WorldPacket::from_bytes(packet).opcode_raw()
            })
        }
    }

    impl BattlepaySession for DummySession {
        fn send_packet<P: ServerPacket>(&self, packet: &P) {
            self.sent.borrow_mut().push(packet.to_bytes());
        }
    }

    #[test]
    fn send_empty_product_list_writes_packet() {
        let session = DummySession::new();
        send_empty_product_list(&session);
        assert_eq!(session.sent.borrow().len(), 1);
        assert_eq!(session.last_opcode(), Some(ServerOpcodes::BattlePayGetProductListResponse.to_u16().unwrap()));
    }

    #[test]
    fn send_start_purchase_response_writes_packet() {
        let session = DummySession::new();
        send_start_purchase_response(&session, 123, 456, 0);
        assert_eq!(session.sent.borrow().len(), 1);
        assert_eq!(session.last_opcode(), Some(ServerOpcodes::BattlePayStartPurchaseResponse.to_u16().unwrap()));
    }

    #[test]
    fn send_product_list_response_writes_packet() {
        let session = DummySession::new();
        send_product_list_response(
            &session,
            1,
            vec![ProductInfo::default()],
            vec![Product::default()],
            vec![Group::default()],
            vec![Shop::default()],
        );
        assert_eq!(session.sent.borrow().len(), 1);
        assert_eq!(session.last_opcode(), Some(ServerOpcodes::BattlePayGetProductListResponse.to_u16().unwrap()));
    }

    #[test]
    fn send_purchase_list_response_writes_packet() {
        let session = DummySession::new();
        send_purchase_list_response(&session, 0, vec![Purchase::default()]);
        assert_eq!(session.sent.borrow().len(), 1);
        assert_eq!(session.last_opcode(), Some(ServerOpcodes::BattlePayGetPurchaseListResponse.to_u16().unwrap()));
    }
}
