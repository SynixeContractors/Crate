#include "..\script_component.hpp"

/*
 * Author: Crate
 * Server side of the Zeus restore. Points the saved device ID at the target object and asks the database for the saved state.
 * The state is applied when the callback arrives in ae3_ext.
 *
 * Arguments:
 * 0: _id <STRING> - Saved device Crate ID
 * 1: _target <OBJECT> - Object to restore onto
 * 2: _silent <BOOL> - (Optional, default false) Don't warn if there is no saved state for this ID
 *
 * Return Value:
 * None
 *
 * Public: No
 */

params ["_id", "_target", ["_silent", false]];

if (_id == "" || {isNull _target}) exitWith {};
if (GVAR(ae3_restores) hasKey _id) exitWith {
    WARNING_1("A restore for AE3 device %1 is already in progress",_id);
};
if ((allMissionObjects "All") findIf { _x != _target && {(_x getVariable [QGVAR(id), ""]) == _id} } != -1) exitWith {
    WARNING_1("AE3 device %1 is already live in the world, not restoring",_id);
};

GVAR(ae3_restores) set [_id, [_target, _target getVariable [QGVAR(id), ""], _silent]];
_target setVariable [QGVAR(id), _id, true];
_target setVariable [QGVAR(ae3_pending), true, true];

EXTCALL("campaigns:ae3:restore",[ARR_1(_id)]);
