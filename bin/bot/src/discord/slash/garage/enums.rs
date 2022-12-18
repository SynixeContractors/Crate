pub enum GarageCommands {
    View,
    PurchaseVehicle,
    PurchaseAddon,
    Attach,
    Detach,
}

impl GarageCommands {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "view" => Some(Self::View),
            "purchase_vehicle" => Some(Self::PurchaseVehicle),
            "purchase_addon" => Some(Self::PurchaseAddon),
            "attach" => Some(Self::Attach),
            "detach" => Some(Self::Detach),
            _ => None,
        }
    }
}

pub enum GarageSubCommands {
    Vehicle,
    Addon,
    Attach,
}

impl GarageSubCommands {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "vehicle" => Some(Self::Vehicle),
            "addon" => Some(Self::Addon),
            "attach" => Some(Self::Attach),
            _ => None,
        }
    }
}
