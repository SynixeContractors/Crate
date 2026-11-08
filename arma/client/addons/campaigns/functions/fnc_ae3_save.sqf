#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Saves the AE3 state of a device or flash drive to the dedicated AE3 table. Runs from the object tick.
 * Devices use AE3's captureState. Flash drives save the AE3 variables stored on the object.
 *
 * Arguments:
 * 0: _object <OBJECT> - Object to check
 * 1: _id <STRING> - Crate ID of the object
 *
 * Return Value:
 * None
 *
 * Public: No
 */

params ["_object", "_id"];

if (isNil "AE3_armaos_fnc_laptop_captureState") exitWith {};
if (_object getVariable [QGVAR(ae3_pending), false]) exitWith {};
if (time < (_object getVariable [QGVAR(ae3_nextUpdate), -1])) exitWith {};
_object setVariable [QGVAR(ae3_nextUpdate), time + 10];

private _kind = "";
if (getText (configOf _object >> "ae3_item") != "") then {
    _kind = "flashdrive";
} else {
    if (
        _object getVariable ["AE3_cap_hasTerminal", false]
        || {_object getVariable ["AE3_cap_isRouter", false]}
        || {isClass (configOf _object >> "AE3_USB_Interface")}
    ) then { _kind = "device" };
};
if (_kind == "") exitWith {};

// A device/flashdrive object is checked once per life for state carried over from a previous one,
// e.g. redeployed from inventory or recreated after a restart, before any new state is captured
if !(_object getVariable [QGVAR(ae3_initChecked), false]) then {
    _object setVariable [QGVAR(ae3_initChecked), true];
    [_id, _object, true] call FUNC(ae3_restore);
    if (_object getVariable [QGVAR(ae3_pending), false]) exitWith {};
};

private _values = if (_kind == "device") then {
    [_object] call AE3_armaos_fnc_laptop_captureState
} else {
    // Crate's own variables are skipped so they can't overwrite the ID of the restored object
    private _ignored = [QGVAR(id), QGVAR(last), QGVAR(nextUpdate), QGVAR(cargo), QGVAR(ae3_last), QGVAR(ae3_nextUpdate), QGVAR(ae3_pending), QGVAR(ae3_initChecked)];
    private _vars = createHashMap;
    {
        if !(_x in _ignored) then {
            _vars set [_x, _object getVariable _x];
        };
    } forEach allVariables _object;
    _vars set ["AE3_OBJECT_TYPE", typeOf _object];
    _vars
};

// AE3's own captureState never includes power state (it always leaves the target's power alone),
// so it is captured separately here and reapplied explicitly in ae3_applyWait
if (_kind == "device") then {
    _values set ["AE3_power_powerState", _object getVariable ["AE3_power_powerState", 0]];
};

private _state = [_kind, [_values] call FUNC(ae3_encode)];
if (_state isEqualTo (_object getVariable [QGVAR(ae3_last), []])) exitWith {};

_object setVariable [QGVAR(ae3_last), _state];

// Sent in chunks so a large filesystem stays under the callExtension size limit
private _text = str _state;
private _size = 4000;
private _offset = 0;
private _chunk = _text select [0, _size];
while {_chunk != ""} do {
    EXTCALL("campaigns:ae3:part",[ARR_2(_id,_chunk)]);
    _offset = _offset + _size;
    _chunk = _text select [_offset, _size];
};
EXTCALL("campaigns:ae3:save",[ARR_2(_id,typeOf _object)]);
