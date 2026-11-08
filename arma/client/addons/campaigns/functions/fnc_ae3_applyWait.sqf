#include "..\script_component.hpp"

/*
 * Author: Crate
 * Per-frame handler that waits for AE3 to finish initializing a device, then applies its saved state.
 *
 * Arguments:
 * 0: _args <ARRAY> - [object, values, deadline]
 * 1: _handle <NUMBER> - Per-frame handler ID
 *
 * Return Value:
 * None
 *
 * Public: No
 */

params ["_args", "_handle"];
_args params ["_object", "_values", "_deadline"];

if (isNull _object) exitWith {
    [_handle] call CBA_fnc_removePerFrameHandler;
};

private _ready = _object getVariable ["AE3_filesystemReady", false];
if (!_ready && {time < _deadline}) exitWith {};

[_handle] call CBA_fnc_removePerFrameHandler;
_object setVariable [QGVAR(ae3_pending), false, true];

if (!_ready) exitWith {
    WARNING_1("AE3 device %1 did not finish initializing, its saved state was not applied",_object);
};

[_object, _values] call AE3_armaos_fnc_laptop_applyState;

// applyState leaves the target's own power state alone, so it's restored explicitly here using the
// real power functions (rather than a raw setVariable) so animations, sounds and textures stay in sync
switch (_values getOrDefault ["AE3_power_powerState", 0]) do {
    case 1: { [_object] call AE3_power_fnc_turnOnDevice; };
    case 2: { [_object] call AE3_power_fnc_standbyDevice; };
};
