#include "..\script_component.hpp"

/*
 * Author: Crate
 * Converts an AE3 state value into a plain array structure that survives the trip to the database and back.
 * Tagged arrays are used for container types: ["A", array], ["H", [[key, value]...]], ["O", crateId], ["C", code], ["N"].
 *
 * Arguments:
 * 0: _value <ANY> - Value to encode
 *
 * Return Value:
 * <ANY> - Encoded value
 *
 * Public: No
 */

params ["_value"];

switch (typeName _value) do {
    case "ARRAY": {
        ["A", _value apply { [_x] call FUNC(ae3_encode) }]
    };
    case "HASHMAP": {
        ["H", (keys _value) apply { [_x, [_value get _x] call FUNC(ae3_encode)] }]
    };
    case "OBJECT": {
        if (isNull _value) exitWith { ["O", ""] };
        private _id = _value getVariable [QGVAR(id), ""];
        if (_id == "") then {
            // Network parents may not have been seen by the object tick yet, so they get an ID here
            EXTFUNC("uuid");
            _id = _ext_res select 0;
            _value setVariable [QGVAR(id), _id, true];
            GVAR(objects_ids) pushBackUnique _id;
        };
        ["O", _id]
    };
    case "CODE": {
        ["C", str _value]
    };
    case "STRING"; case "SCALAR"; case "BOOL": {
        _value
    };
    default {
        ["N"]
    };
};
