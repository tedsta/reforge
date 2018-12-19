use ggez::graphics::FontId;

use asset_store::AssetStore;
use module::ModelStore;
use net::Client;
use sector_data::SectorData;

pub struct ReforgeClientContext {
    pub font: FontId,
    pub asset_store: AssetStore,
    pub model_store: ModelStore,
    pub client: Client,
    pub sectors: Vec<SectorData>,
}
