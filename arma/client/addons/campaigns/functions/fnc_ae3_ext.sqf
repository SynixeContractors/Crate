#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
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
        (GVAR(ae3_restores) getOrDefault [_id, []]) params [["_target", objNull], ["_previousId", ""], ["_silent", false]];
        GVAR(ae3_restores) deleteAt _id;

        if (isNull _target) exitWith {
            WARNING_1("No pending target for AE3 device %1",_id);
        };

        if (_class == "" || {_state isEqualTo ""}) exitWith {
            if !(_silent) then {
                WARNING_1("AE3 device %1 has no saved state for this object, reverting the ID",_id);
            };
            _target setVariable [QGVAR(id), _previousId, true];
            _target setVariable [QGVAR(ae3_pending), false, true];
        };

        if (typeOf _target != _class) exitWith {
            WARNING_1("AE3 device %1 was saved from a different object type, reverting the ID",_id);
            _target setVariable [QGVAR(id), _previousId, true];
            _target setVariable [QGVAR(ae3_pending), false, true];
        };

        [_target, _state] call FUNC(ae3_apply);
    };
    case "itemPart": {
        (parseSimpleArray _data) params ["_id", "_chunk"];
        GVAR(ae3_item_parts) set [_id, (GVAR(ae3_item_parts) getOrDefault [_id, []]) + [_chunk]];
    };
    case "itemEnd": {
        (parseSimpleArray _data) params ["_id", "_class"];
        private _text = (GVAR(ae3_item_parts) getOrDefault [_id, []]) joinString "";
        GVAR(ae3_item_parts) deleteAt _id;
        GVAR(ae3_item_states) set [_id, [_class, _text]];
    };
    case "itemLoad": {
        (parseSimpleArray _data) params ["_item", "_id", "_class"];
        // AE3 stores laptops and flash drives in separate buffers, told apart the same way ae3_save does
        private _isFlashdrive = [_class] call FUNC(ae3_isFlashdrive);
        private _bufferVar = ["AE3_LAPTOP_ITEM", "AE3_ITEM"] select _isFlashdrive;
        private _buffer = missionNamespace getVariable [_bufferVar, createHashMap];
        // A minimal namespace is enough to get past AE3's deploy checks; the Crate ID carried on it
        // lets ae3_save's own redeploy check pull down the real saved state once the object exists.
        private _itemNamespace = createHashMap;
        _itemNamespace set ["AE3_OBJECT_TYPE", _class];
        _itemNamespace set [toLower QGVAR(id), _id];

        // A packed item's saved state goes into it now, so it's ready exactly as AE3 expects when deployed. A flash
        // drive is marked as initialized, as AE3 mounts its filesystem as soon as it's connected and restoring after
        // that would leave the computer with an empty copy. A laptop isn't marked, so ae3_save still restores it
        // once deployed to reapply its power state, which AE3 doesn't carry across deploys.
        (GVAR(ae3_item_states) getOrDefault [_id, []]) params [["_savedClass", ""], ["_savedState", ""]];
        GVAR(ae3_item_states) deleteAt _id;
        if (_savedState == "") then {
            WARNING_1("No saved AE3 state for packed item %1",_id);
        };
        if (_savedState != "") then {
            private _encoded = (parseSimpleArray _savedState) select 1;
            private _values = [_encoded] call FUNC(ae3_decode);
            {
                if ((toLower _x) find "ae3_" == 0) then {
                    _itemNamespace set [_x, _values get _x];
                };
            } forEach keys _values;
            if (_isFlashdrive) then {
                _itemNamespace set [toLower QGVAR(ae3_initChecked), true];
            } else {
                // The packed item carries no power state, so the saved one is kept for items_tick to store alongside it
                GVAR(ae3_power) set [_id, _values getOrDefault ["AE3_power_powerState", 0]];
            };
        };

        _buffer set [_item, _itemNamespace];
        missionNamespace setVariable [_bufferVar, _buffer, false];
        // Registered as already sent, as the snapshot the packed-item tick compares against
        private _known = GVAR(ae3_packed_known) getOrDefault [_bufferVar, createHashMap];
        _known set [_item, [_id, [["device", "flashdrive"] select _isFlashdrive, _itemNamespace] call FUNC(ae3_state)]];
        GVAR(ae3_packed_known) set [_bufferVar, _known];
    };
};
