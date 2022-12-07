#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(EGVAR(gear_main,enabled)) exitWith {};

[{
    call FUNC(create_actions);
}] call CBA_fnc_execNextFrame;

["ace_arsenal_displayOpened", FUNC(arsenal_handle_DisplayOpened)] call CBA_fnc_addEventHandler;
["ace_arsenal_displayClosed", FUNC(arsenal_handle_DisplayClosed)] call CBA_fnc_addEventHandler;
["ace_arsenal_leftPanelFilled", FUNC(arsenal_handle_LeftPanelFilled)] call CBA_fnc_addEventHandler;
["ace_arsenal_rightPanelFilled", FUNC(arsenal_handle_RightPanelFilled)] call CBA_fnc_addEventHandler;
["ace_arsenal_loadoutsListFilled", FUNC(arsenal_handle_LoadoutsListFilled)] call CBA_fnc_addEventHandler;
