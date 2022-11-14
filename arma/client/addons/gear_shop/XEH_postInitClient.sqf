#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(EGVAR(gear_main,enabled)) exitWith {};

[{
	call FUNC(create_actions);
}] call CBA_fnc_execNextFrame;
