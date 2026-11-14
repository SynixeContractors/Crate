#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
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

// AE3's computer turn-on chain ends without returning a value, so its power wrapper never sets the power state and
// leaves the power mutex stuck. The original is run and success is always reported.
if (isNil {_object getVariable QGVAR(ae3_turnOnOriginal)}) then {
    private _turnOn = _object getVariable ["AE3_power_fnc_turnOn", {}];
    if (_turnOn isNotEqualTo {}) then {
        _object setVariable [QGVAR(ae3_turnOnOriginal), _turnOn];
        _object setVariable ["AE3_power_fnc_turnOn", {
            private _original = (_this select 0) getVariable [QGVAR(ae3_turnOnOriginal), {}];
            _this call _original;
            true
        }];
    };
};

// AE3 always deploys with the lid open, and its power actions need it open, so a lid saved closed is reopened.
// The power mutex is a runtime lock set while a power action runs, so a saved lock would block power actions for good.
if (_object getVariable ["AE3_interaction_closeState", 0] == 1) then {
    _object setVariable ["AE3_interaction_closeState", 0, true];
};
if (_object getVariable ["AE3_power_mutex", false]) then {
    _object setVariable ["AE3_power_mutex", false, true];
};

// applyState leaves the target's own power state alone, so it's restored explicitly here using the
// real power functions (rather than a raw setVariable) so animations, sounds and textures stay in sync
switch (_values getOrDefault ["AE3_power_powerState", 0]) do {
    case 0: {
        // A deployed laptop has no power state until AE3's server-only deploy path sets one, so an Off save leaves it nil
        if (isNil {_object getVariable "AE3_power_powerState"}) then {
            _object setVariable ["AE3_power_powerState", 0, true];
        };
    };
    case 1: { [_object] call AE3_power_fnc_turnOnDevice; };
    case 2: { [_object] call AE3_power_fnc_standbyDevice; };
};
