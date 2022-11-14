#include "script_component.hpp"

if !(GVAR(enabled)) exitWith {};

private _action = [QGVAR(box), "Shop", "", {
	[_target] call FUNC(box_open);
}, {
	EGVAR(shop,items) isNotEqualTo [] // ARMA TODO
}] call ace_interact_menu_fnc_createAction;

{
	if !(_x getVariable [QGVAR(has_action), false]) then {
		[_x, 0, ["ACE_MainActions"], _action] call ace_interact_menu_fnc_addActionToObject;
		_x setVariable [QGVAR(has_action), true];
	};
} forEach GVAR(boxes);
