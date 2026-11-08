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

(parseSimpleArray _text) params ["", "_encoded"];
private _values = [_encoded] call FUNC(ae3_decode);

// Defends against state saved before this filter existed, or anything that slips past ae3_save in
// the future: only AE3's own variables are safe to apply to a (re)created object, see ae3_save
private _ae3Values = createHashMap;
{
    if ((toLower _x) find "ae3_" == 0) then {
        _ae3Values set [_x, _y];
    };
} forEach _values;
_values = _ae3Values;

// Decided by the object, not the saved kind, so states saved before laptops were told apart still apply correctly
if ([typeOf _object] call FUNC(ae3_isFlashdrive)) exitWith {
    {
        _object setVariable [_x, _values get _x];
    } forEach keys _values;
    _object setVariable [QGVAR(ae3_pending), false, true];
};

// AE3 can't attach a network parent that isn't in the world, so it's dropped rather than leaving a broken link
if ((_values getOrDefault ["ae3_network_parent", objNull]) isEqualTo objNull) then {
    _values deleteAt "ae3_network_parent";
};

[_object] call AE3_armaos_fnc_device_ensureInit;
[FUNC(ae3_applyWait), 0.5, [_object, _values, time + 30]] call CBA_fnc_addPerFrameHandler;
