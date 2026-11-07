#include "..\script_component.hpp"

/*
 * Author: Crate
 * Handles callbacks from the AE3 extension commands.
 *
 * Arguments:
 * 0: _func <STRING> - Callback name
 * 1: _data <STRING> - Callback data
 *
 * Return Value:
 * None
 *
 * Public: No
 */

params ["_func", "_data"];

switch (_func) do {
    case "part": {
        (parseSimpleArray _data) params ["_id", "_chunk"];
        GVAR(ae3_parts) set [_id, (GVAR(ae3_parts) getOrDefault [_id, []]) + [_chunk]];
    };
    case "end": {
        (parseSimpleArray _data) params ["_id", "_class"];
        private _text = (GVAR(ae3_parts) getOrDefault [_id, []]) joinString "";
        GVAR(ae3_parts) deleteAt _id;
        GVAR(ae3_states) set [_id, [_class, _text]];
    };
    case "restore": {
        (parseSimpleArray _data) params ["_id"];
        (GVAR(ae3_states) getOrDefault [_id, []]) params [["_class", ""], ["_state", ""]];
        GVAR(ae3_states) deleteAt _id;
        (GVAR(ae3_restores) getOrDefault [_id, []]) params [["_target", objNull], ["_previousId", ""]];
        GVAR(ae3_restores) deleteAt _id;

        if (isNull _target) exitWith {
            WARNING_1("No pending target for AE3 device %1",_id);
        };

        if (_class == "" || {_state isEqualTo ""} || {typeOf _target != _class}) exitWith {
            WARNING_1("AE3 device %1 has no saved state for this object, reverting the ID",_id);
            _target setVariable [QGVAR(id), _previousId, true];
            _target setVariable [QGVAR(ae3_pending), false, true];
        };

        [_target, _state] call FUNC(ae3_apply);
    };
};
