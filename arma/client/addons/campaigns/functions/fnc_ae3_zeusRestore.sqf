#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Zeus (ZEN custom module) entry point. Attach the module to a device and enter the saved device ID to restore it.
 * Runs on the machine where the module is placed and forwards the request to the server.
 *
 * Arguments:
 * 0: _position <ARRAY> - Module position (ASL)
 * 1: _target <OBJECT> - Attached object
 *
 * Return Value:
 * None
 *
 * Public: No
 */

params [["_position", [0,0,0], [[]]], ["_target", objNull, [objNull]]];

if (isNull _target) exitWith {
    hint "Attach the restore module to the device you want to restore.";
};

private _onConfirm = {
    params ["_values", "_args"];
    _args params ["_target"];
    private _id = trim (_values select 0);
    if (_id == "") exitWith {};
    [_id, _target] remoteExecCall [QFUNC(ae3_restore), 2];
};

[
    "Restore AE3 Device",
    [
        ["EDIT", "Saved device ID", ["", {_this}, 1]]
    ],
    _onConfirm,
    {},
    [_target],
    "crate_campaigns_ae3_restore"
] call zen_dialog_fnc_create;
