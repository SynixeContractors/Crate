#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Applies a saved AE3 state to a live object. Devices are applied once AE3 has finished initializing them.
 *
 * Arguments:
 * 0: _object <OBJECT> - Target object
 * 1: _text <STRING> - Saved state as stored by ae3_save: the SQF string of [kind, encodedValues]
 *
 * Return Value:
 * None
 *
 * Public: No
 */

params ["_object", "_text"];

(parseSimpleArray _text) params ["_kind", "_encoded"];
private _values = [_encoded] call FUNC(ae3_decode);

if (_kind == "flashdrive") exitWith {
    {
        _object setVariable [_x, _values get _x];
    } forEach keys _values;
    _object setVariable [QGVAR(ae3_pending), false, true];
};

// AE3 can't attach a network parent that isn't in the world, so it's dropped rather than leaving a broken link
if ((_values getOrDefault ["AE3_network_parent", objNull]) isEqualTo objNull) then {
    _values deleteAt "AE3_network_parent";
};

[_object] call AE3_armaos_fnc_device_ensureInit;
[FUNC(ae3_applyWait), 0.5, [_object, _values, time + 30]] call CBA_fnc_addPerFrameHandler;
