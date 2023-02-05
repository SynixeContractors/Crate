#[derive(Debug)]
pub enum Command {
    View,
    PurchaseVehicle,
    PurchaseAddon,
    Attach,
    Detach,
    Spawn,
}

impl Command {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "view" => Some(Self::View),
            "purchase_vehicle" => Some(Self::PurchaseVehicle),
            "purchase_addon" => Some(Self::PurchaseAddon),
            "attach" => Some(Self::Attach),
            "detach" => Some(Self::Detach),
            "spawn" => Some(Self::Spawn),
            _ => None,
        }
    }
}

pub enum AssetFilter {
    Vehicle(Option<String>),
    Addon(Option<String>),
}

impl AssetFilter {
    pub fn search(&self) -> Option<String> {
        match &self {
            Self::Vehicle(s) | Self::Addon(s) => s.clone(),
        }
    }
}
