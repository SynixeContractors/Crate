#include "script_component.hpp"

params ["_itemCfg"];

([configName _itemCfg, true] call FUNC(shop_item_price)) params ["_personal"];

_personal
