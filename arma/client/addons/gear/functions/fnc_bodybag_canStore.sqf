#include "..\script_component.hpp"

params ["_bodyBag"];

if !(GVAR(enabled)) exitWith { false };
if (GVAR(readOnly)) exitWith { false };

if (_bodyBag getVariable [QEGVAR(bodybag,id), ""] == "") exitWith { false };
if (count ([_bodybag] call EFUNC(bodybag,contents)) == 0) exitWith { false };

private _objects = GVAR(shop_boxes) select { _x distance _bodyBag < 15 };
count _objects != 0
