#include "script_component.hpp"

params ["_itemCfg"];

-1 * ([configName _itemCfg] call FUNC(shop_item_owned))
