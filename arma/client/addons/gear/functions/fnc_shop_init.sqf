#include "script_component.hpp"

if !(GVAR(enabled)) exitWith {};
if !(GVAR(shop_enabled)) exitWith {};

private _action = [QGVAR(box), "Shop", "", {
    [_target] call FUNC(shop_open);
}, {
    GVAR(enabled) && {!(player getVariable [QGVAR(shop_open), false])} && {count GVAR(shop_items) > 0}
}] call ace_interact_menu_fnc_createAction;

{
    if !(_x getVariable [QGVAR(has_action), false]) then {
        [_x, 0, ["ACE_MainActions"], _action] call ace_interact_menu_fnc_addActionToObject;
        _x setVariable [QGVAR(has_action), true];
    };
} forEach GVAR(shop_boxes);
