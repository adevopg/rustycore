// BattlePay packet definitions ported from C++ BattlePayPackets

use wow_constants::ServerOpcodes;
use wow_core::ObjectGuid;
use crate::{ServerPacket, WorldPacket};

// ---------- Data structures ----------

#[derive(Debug, Clone, Default)]
pub struct Visual {
    pub name: String,
    pub display_id: u32,
    pub visual_id: u32,
    pub unk: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DisplayInfo {
    pub creature_display_id: Option<u32>,
    pub visual_id: Option<u32>,
    pub name1: String,
    pub name2: String,
    pub name3: String,
    pub name4: String,
    pub name5: String,
    pub name6: String,
    pub name7: String,
    pub flags: Option<u32>,
    pub unk1: Option<u32>,
    pub unk2: Option<u32>,
    pub unk3: Option<u32>,
    pub unk_int1: u32,
    pub unk_int2: u32,
    pub unk_int3: u32,
    pub disable_listing: Option<u8>,
    pub disable_buy: Option<u8>,
    pub name_color_index: Option<u8>,
    pub script_name: String,
    pub comment: String,
    pub visuals: Vec<Visual>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductInfo {
    pub product_id: u32,
    pub normal_price_fixed_point: u64,
    pub current_price_fixed_point: u64,
    pub product_ids: Vec<u32>,
    pub unk1: u32,
    pub unk2: u32,
    pub unk_ints: Vec<u32>,
    pub unk3: u32,
    pub choice_type: u32,
    pub display: Option<DisplayInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductItem {
    pub id: u32,
    pub unk_byte: u8,
    pub item_id: u32,
    pub quantity: u32,
    pub unk_int1: u32,
    pub unk_int2: u32,
    pub is_pet: bool,
    pub pet_result: Option<u32>,
    pub display: Option<DisplayInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct Product {
    pub product_id: u32,
    pub typ: u8,
    pub flags: u32,
    pub unk1: u32,
    pub display_id: u32,
    pub item_id: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub unk7: u32,
    pub unk8: u32,
    pub unk9: u32,
    pub unk_string: String,
    pub unk_bit: bool,
    pub unk_bits: Option<u32>,
    pub items: Vec<ProductItem>,
    pub display: Option<DisplayInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct Group {
    pub group_id: u32,
    pub icon_file_data_id: u32,
    pub display_type: u8,
    pub ordering: u32,
    pub unk: u32,
    pub main_group_id: u32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct Shop {
    pub entry_id: u32,
    pub group_id: u32,
    pub product_id: u32,
    pub ordering: u32,
    pub vas_service_type: u32,
    pub store_delivery_type: u8,
    pub display: Option<DisplayInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct DistributionObject {
    pub product: Option<Product>,
    pub target_player: ObjectGuid,
    pub distribution_id: u64,
    pub purchase_id: u64,
    pub status: u32,
    pub product_id: u32,
    pub target_virtual_realm: u32,
    pub target_native_realm: u32,
    pub revoked: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Purchase {
    pub purchase_id: u64,
    pub unk_long: u64,
    pub unk_long2: u64,
    pub status: u32,
    pub result_code: u32,
    pub product_id: u32,
    pub unk_int: u32,
    pub wallet_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct VasPurchase {
    pub item_ids: Vec<u32>,
    pub player_guid: ObjectGuid,
    pub unk_long: u64,
    pub unk_int: u32,
    pub unk_int2: u32,
}

// ---------- Helper writers (porting C++ operator<<) ----------

fn write_display_info(pkt: &mut WorldPacket, info: &DisplayInfo) {
    pkt.write_bit(info.creature_display_id.is_some());
    pkt.write_bit(info.visual_id.is_some());
    pkt.write_bits(info.name1.len() as u32, 10);
    pkt.write_bits(info.name2.len() as u32, 10);
    pkt.write_bits(info.name3.len() as u32, 13);
    pkt.write_bits(info.name4.len() as u32, 13);
    pkt.write_bits(info.name5.len() as u32, 13);
    pkt.write_bit(info.flags.is_some());
    pkt.write_bit(info.unk1.is_some());
    pkt.write_bit(info.unk2.is_some());
    pkt.write_bit(info.unk3.is_some());
    pkt.write_bits(info.name6.len() as u32, 13);
    pkt.write_bits(info.name7.len() as u32, 13);
    pkt.flush_bits();

    pkt.write_uint32(info.visuals.len() as u32);
    pkt.write_uint32(info.unk_int1);
    pkt.write_uint32(info.unk_int2);
    pkt.write_uint32(info.unk_int3);

    if let Some(v) = info.creature_display_id {
        pkt.write_uint32(v);
    }
    if let Some(v) = info.visual_id {
        pkt.write_uint32(v);
    }

    pkt.write_string(&info.name1);
    pkt.write_string(&info.name2);
    pkt.write_string(&info.name3);
    pkt.write_string(&info.name4);
    pkt.write_string(&info.name5);

    if let Some(f) = info.flags {
        pkt.write_uint32(f);
    }
    if let Some(u) = info.unk1 {
        pkt.write_uint32(u);
    }
    if let Some(u) = info.unk2 {
        pkt.write_uint32(u);
    }
    if let Some(u) = info.unk3 {
        pkt.write_uint32(u);
    }

    pkt.write_string(&info.name6);
    pkt.write_string(&info.name7);

    for visual in &info.visuals {
        pkt.write_bits(visual.name.len() as u32, 10);
        pkt.flush_bits();
        pkt.write_uint32(visual.display_id);
        pkt.write_uint32(visual.visual_id);
        pkt.write_uint32(visual.unk);
        pkt.write_string(&visual.name);
    }
}

fn write_product_info(pkt: &mut WorldPacket, p: &ProductInfo) {
    pkt.write_uint32(p.product_id);
    pkt.write_uint64(p.normal_price_fixed_point);
    pkt.write_uint64(p.current_price_fixed_point);
    pkt.write_uint32(p.product_ids.len() as u32);
    pkt.write_uint32(p.unk1);
    pkt.write_uint32(p.unk2);
    pkt.write_uint32(p.unk_ints.len() as u32);
    pkt.write_uint32(p.unk3);
    for id in &p.product_ids {
        pkt.write_uint32(*id);
    }
    for id in &p.unk_ints {
        pkt.write_uint32(*id);
    }
    pkt.write_bits(p.choice_type, 7);
    let wrote = p.display.is_some();
    pkt.write_bit(wrote);
    pkt.flush_bits();
    if wrote {
        write_display_info(pkt, p.display.as_ref().unwrap());
    }
}

fn write_product_item(pkt: &mut WorldPacket, p: &ProductItem) {
    pkt.write_uint32(p.id);
    pkt.write_uint8(p.unk_byte);
    pkt.write_uint32(p.item_id);
    pkt.write_uint32(p.quantity);
    pkt.write_uint32(p.unk_int1);
    pkt.write_uint32(p.unk_int2);
    pkt.write_bit(p.is_pet);
    pkt.write_bit(p.pet_result.is_some());
    pkt.write_bit(p.display.is_some());
    if let Some(pr) = p.pet_result {
        pkt.write_bits(pr, 4);
    }
    pkt.flush_bits();
    if let Some(d) = &p.display {
        write_display_info(pkt, d);
    }
}

fn write_product(pkt: &mut WorldPacket, p: &Product) {
    pkt.write_uint32(p.product_id);
    pkt.write_uint8(p.typ);
    pkt.write_uint32(p.flags);
    pkt.write_uint32(p.unk1);
    pkt.write_uint32(p.display_id);
    pkt.write_uint32(p.item_id);
    pkt.write_uint32(p.unk4);
    pkt.write_uint32(p.unk5);
    pkt.write_uint32(p.unk6);
    pkt.write_uint32(p.unk7);
    pkt.write_uint32(p.unk8);
    pkt.write_uint32(p.unk9);
    pkt.write_bits(p.unk_string.len() as u32, 8);
    pkt.write_bit(p.unk_bit);
    pkt.write_bit(p.unk_bits.is_some());
    pkt.write_bits(p.items.len() as u32, 7);
    pkt.write_bit(p.display.is_some());
    if let Some(b) = p.unk_bits {
        pkt.write_bits(b, 4);
    }
    pkt.flush_bits();

    for item in &p.items {
        write_product_item(pkt, item);
    }

    pkt.write_string(&p.unk_string);

    if let Some(d) = &p.display {
        write_display_info(pkt, d);
    }
}

fn write_group(pkt: &mut WorldPacket, g: &Group) {
    pkt.write_uint32(g.group_id);
    pkt.write_uint32(g.icon_file_data_id);
    pkt.write_uint8(g.display_type);
    pkt.write_uint32(g.ordering);
    pkt.write_uint32(g.unk);
    pkt.write_uint32(g.main_group_id);
    pkt.write_bits(g.name.len() as u32, 8);
    pkt.write_bits((g.description.len().saturating_add(1)) as u32, 24);
    pkt.flush_bits();
    pkt.write_string(&g.name);
    if !g.description.is_empty() {
        pkt.write_string(&g.description);
    }
}

fn write_shop(pkt: &mut WorldPacket, s: &Shop) {
    pkt.write_uint32(s.entry_id);
    pkt.write_uint32(s.group_id);
    pkt.write_uint32(s.product_id);
    pkt.write_uint32(s.ordering);
    pkt.write_uint32(s.vas_service_type);
    pkt.write_uint8(s.store_delivery_type);
    pkt.flush_bits();
    pkt.write_bit(s.display.is_some());
    if let Some(d) = &s.display {
        pkt.flush_bits();
        write_display_info(pkt, d);
    }
}

fn write_distribution_object(pkt: &mut WorldPacket, o: &DistributionObject) {
    pkt.write_uint64(o.distribution_id);
    pkt.write_uint32(o.status);
    pkt.write_uint32(o.product_id);
    pkt.write_packed_guid(&o.target_player);
    pkt.write_uint32(o.target_virtual_realm);
    pkt.write_uint32(o.target_native_realm);
    pkt.write_uint64(o.purchase_id);
    pkt.write_bit(o.product.is_some());
    pkt.write_bit(o.revoked);
    pkt.flush_bits();
    if let Some(prod) = &o.product {
        write_product(pkt, prod);
    }
}

fn write_purchase(pkt: &mut WorldPacket, p: &Purchase) {
    pkt.write_uint64(p.purchase_id);
    pkt.write_uint32(p.status);
    pkt.write_uint32(p.result_code);
    pkt.write_uint32(p.product_id);
    pkt.write_uint64(p.unk_long);
    pkt.write_uint64(p.unk_long2);
    pkt.write_uint32(p.unk_int);

    pkt.write_bits(p.wallet_name.len() as u32, 8);
    pkt.write_string(&p.wallet_name);
}

fn write_vas_purchase(pkt: &mut WorldPacket, v: &VasPurchase) {
    pkt.write_packed_guid(&v.player_guid);
    pkt.write_uint32(v.unk_int);
    pkt.write_uint32(v.unk_int2);
    pkt.write_uint64(v.unk_long);
    pkt.write_bits(v.item_ids.len() as u32, 2);
    pkt.flush_bits();
    for id in &v.item_ids {
        pkt.write_uint32(*id);
    }
}

// ---------- Server packet implementations ----------

pub struct PurchaseListResponse {
    pub result: u32,
    pub purchase: Vec<Purchase>,
}

impl ServerPacket for PurchaseListResponse {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayGetPurchaseListResponse;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint32(self.result);
        pkt.write_uint32(self.purchase.len() as u32);
        for p in &self.purchase {
            write_purchase(pkt, p);
        }
    }
}

pub struct DistributionListResponse {
    pub result: u32,
    pub distribution_object: Vec<DistributionObject>,
}

impl ServerPacket for DistributionListResponse {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayGetDistributionListResponse;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint32(self.result);
        pkt.write_bits(self.distribution_object.len() as u32, 11);
        pkt.flush_bits();
        for o in &self.distribution_object {
            write_distribution_object(pkt, o);
        }
    }
}

pub struct DistributionUpdate {
    pub distribution_object: DistributionObject,
}

impl ServerPacket for DistributionUpdate {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayDistributionUpdate;

    fn write(&self, pkt: &mut WorldPacket) {
        write_distribution_object(pkt, &self.distribution_object);
    }
}

pub struct ProductListResponse {
    pub result: u32,
    pub currency_id: u32,
    pub product_infos: Vec<ProductInfo>,
    pub products: Vec<Product>,
    pub product_groups: Vec<Group>,
    pub shops: Vec<Shop>,
}

impl ServerPacket for ProductListResponse {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayGetProductListResponse;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint32(self.result);
        pkt.write_uint32(self.currency_id);
        pkt.write_uint32(self.product_infos.len() as u32);
        pkt.write_uint32(self.products.len() as u32);
        pkt.write_uint32(self.product_groups.len() as u32);
        pkt.write_uint32(self.shops.len() as u32);

        for p in &self.product_infos {
            write_product_info(pkt, p);
        }
        for p in &self.products {
            write_product(pkt, p);
        }
        for g in &self.product_groups {
            write_group(pkt, g);
        }
        for s in &self.shops {
            write_shop(pkt, s);
        }
    }
}

pub struct SyncWowEntitlements {
    pub purchase_count: Vec<u32>,
    pub product: Vec<Product>,
}

impl ServerPacket for SyncWowEntitlements {
    const OPCODE: ServerOpcodes = ServerOpcodes::SyncWowEntitlements;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint32(self.purchase_count.len() as u32);
        pkt.write_uint32(self.product.len() as u32);

        for _ in &self.purchase_count {
            pkt.write_uint32(0);
            pkt.write_uint32(0);
            pkt.write_uint32(0);
            pkt.write_uint32(0);
            pkt.write_bits(0, 7);
            pkt.write_bit(false);
        }

        for product in &self.product {
            pkt.write_uint32(product.product_id);
            pkt.write_uint32(product.typ as u32);
            pkt.write_uint32(product.flags);
            pkt.write_uint32(product.unk1);
            pkt.write_uint32(product.display_id);
            pkt.write_uint32(product.item_id);
            pkt.write_uint32(0);
            pkt.write_uint32(2);
            pkt.write_uint32(0);
            pkt.write_uint32(0);
            pkt.write_uint32(0);
            pkt.write_uint32(0);

            pkt.write_bits(product.unk_string.len() as u32, 8);
            pkt.write_bit(product.unk_bits.is_some());
            pkt.write_bit(product.unk_bit);
            pkt.write_bits(product.items.len() as u32, 7);
            pkt.write_bit(product.display.is_some());
            pkt.write_bit(false);

            if let Some(b) = product.unk_bits {
                pkt.write_bits(b, 4);
            }

            pkt.flush_bits();

            for item in &product.items {
                pkt.write_uint32(item.id);
                pkt.write_uint8(item.unk_byte);
                pkt.write_uint32(item.item_id);
                pkt.write_uint32(item.quantity);
                pkt.write_uint32(item.unk_int1);
                pkt.write_uint32(item.unk_int2);

                pkt.write_bit(item.is_pet);
                pkt.write_bit(item.pet_result.is_some());
                pkt.write_bit(item.display.is_some());

                if let Some(pr) = item.pet_result {
                    pkt.write_bits(pr, 4);
                }

                pkt.flush_bits();

                if let Some(d) = &item.display {
                    write_display_info(pkt, d);
                }
            }

            pkt.write_string(&product.unk_string);

            if let Some(d) = &product.display {
                write_display_info(pkt, d);
            }
        }
    }
}

pub struct StartPurchaseResponse {
    pub purchase_id: u64,
    pub client_token: u32,
    pub purchase_result: u32,
}

impl ServerPacket for StartPurchaseResponse {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayStartPurchaseResponse;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint64(self.purchase_id);
        pkt.write_uint32(self.purchase_result);
        pkt.write_uint32(self.client_token);
    }
}

pub struct BattlePayAckFailedPacket {
    pub purchase_id: u64,
    pub purchase_result: u32,
    pub client_token: u32,
}

impl ServerPacket for BattlePayAckFailedPacket {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayAckFailed;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint64(self.purchase_id);
        pkt.write_uint32(self.purchase_result);
        pkt.write_uint32(self.client_token);
    }
}

pub struct PurchaseUpdate {
    pub purchase: Vec<Purchase>,
}

impl ServerPacket for PurchaseUpdate {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayPurchaseUpdate;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint32(self.purchase.len() as u32);
        for p in &self.purchase {
            write_purchase(pkt, p);
        }
    }
}

pub struct ConfirmPurchase {
    pub purchase_id: u64,
    pub server_token: u32,
}

impl ServerPacket for ConfirmPurchase {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayConfirmPurchase;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint64(self.purchase_id);
        pkt.write_uint32(self.server_token);
    }
}

pub struct DeliveryEnded {
    pub item: Vec<Vec<u8>>,
    pub distribution_id: u64,
}

impl ServerPacket for DeliveryEnded {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayDeliveryEnded;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint64(self.distribution_id);
        pkt.write_int32(self.item.len() as i32);
        for it in &self.item {
            pkt.write_bytes(it);
        }
    }
}

pub struct DeliveryStarted {
    pub distribution_id: u64,
}

impl ServerPacket for DeliveryStarted {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayDeliveryStarted;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint64(self.distribution_id);
    }
}

pub struct UpgradeStarted {
    pub character_guid: ObjectGuid,
}

impl ServerPacket for UpgradeStarted {
    const OPCODE: ServerOpcodes = ServerOpcodes::CharacterUpgradeStarted;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_packed_guid(&self.character_guid);
    }
}

pub struct BattlePayBattlePetDelivered {
    pub display_id: u32,
    pub battle_pet_guid: ObjectGuid,
}

impl ServerPacket for BattlePayBattlePetDelivered {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayBattlePetDelivered;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint32(self.display_id);
        pkt.write_packed_guid(&self.battle_pet_guid);
    }
}

pub struct DisplayPromotion {
    pub promotion_id: u32,
}

impl ServerPacket for DisplayPromotion {
    const OPCODE: ServerOpcodes = ServerOpcodes::DisplayPromotion;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint32(self.promotion_id);
    }
}

pub struct BattlePayStartDistributionAssignToTargetResponse {
    pub distribution_id: u64,
    pub unkint1: u32,
    pub unkint2: u32,
}

impl ServerPacket for BattlePayStartDistributionAssignToTargetResponse {
    const OPCODE: ServerOpcodes = ServerOpcodes::BattlePayStartDistributionAssignToTargetResponse;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_uint64(self.distribution_id);
        pkt.write_uint32(self.unkint1);
        pkt.write_uint32(self.unkint2);
    }
}

pub struct EnumVasPurchaseStatesResponse {
    pub result: u8,
}

impl ServerPacket for EnumVasPurchaseStatesResponse {
    const OPCODE: ServerOpcodes = ServerOpcodes::EnumVasPurchaseStatesResponse;

    fn write(&self, pkt: &mut WorldPacket) {
        pkt.write_bits(self.result as u32, 2);
        pkt.flush_bits();
    }
}

