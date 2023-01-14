#include "script_component.hpp"

params ["_itemCfg"];

[configName _itemCfg] call FUNC(shop_item_owned)
