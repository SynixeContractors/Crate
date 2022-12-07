#include "script_component.hpp"

params ["_display"];

if !(EGVAR(gear_main,enabled)) exitWith {};

if (EGVAR(gear_main,read_only)) exitWith {
    [QGVAR(closing)] call CBA_fnc_localEvent;
};

if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(open), false]) exitWith {};

player setVariable [QGVAR(open), false];
player setVariable [QGVAR(inArsenal), false, true];

[GVAR(balanceHandle)] call CBA_fnc_removePerFrameHandler;

private _items = [[player] call CBA_fnc_getLoadout] call EFUNC(gear_locker,loadout_items);
private _items = [_items] call EFUNC(gear_locker,loadout_remove_owned);
private _cost = [_items] call FUNC(items_cost);

if (_cost == 0) then {
    [QGVAR(closing)] call CBA_fnc_localEvent;
} else {
    systemChat "You do not own all items, reverting changes.";
    [QGVAR(reverting)] call CBA_fnc_localEvent;
};
