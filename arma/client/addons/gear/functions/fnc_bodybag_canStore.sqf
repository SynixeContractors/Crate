#include "..\script_component.hpp"

params ["_bodyBag"];

if !(GVAR(enabled)) exitWith { false };
if (GVAR(readOnly)) exitWith { false };
if (EGVAR(campaigns,loadouts)) exitWith { false };

if (_bodyBag getVariable [QEGVAR(discord,id), ""] == "") exitWith { false };
if (count ([_bodyBag] call FUNC(bodybag_contents)) == 0) exitWith { false };

(GVAR(shop_boxes) findIf { _x distance _bodyBag < 5 }) != -1
