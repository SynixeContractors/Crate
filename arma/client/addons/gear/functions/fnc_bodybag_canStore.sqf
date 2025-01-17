#include "..\script_component.hpp"

params ["_bodyBag"];

if !(GVAR(enabled)) exitWith { false };
if (GVAR(readOnly)) exitWith { false };

if (_bodyBag getVariable [QEGVAR(discord,id), ""] == "") exitWith { false };
if (count ([_bodyBag] call FUNC(bodybag_contents)) == 0) exitWith { false };

private _nearbyShops = GVAR(shop_boxes) select { _x distance _bodyBag < 5 };
count _nearbyShops != 0
