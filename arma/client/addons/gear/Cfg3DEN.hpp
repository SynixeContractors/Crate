class Cfg3DEN {
    class Object {
        class AttributeCategories {
            class ADDON {
                displayName = "Core - Persistent Gear";
                class Attributes {
                    class Shop {
                        displayName = "Shop";
                        tooltip = "Add a shop via ACE Interact";
                        property = QGVAR(attribute_shop);
                        control = "Checkbox";
                        expression = QUOTE(if(_value)then{GVAR(shop_boxes) pushBack _this});
                        defaultValue = QUOTE(false);
                    };
                };
            };
        };
    };
};
