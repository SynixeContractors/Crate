#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Reverses ae3_encode. Object references are resolved to live objects by their Crate ID, objNull when not found.
 *
 * Arguments:
 * 0: _value <ANY> - Encoded value
 *
 * Return Value:
 * <ANY> - Decoded value
 *
 * Public: No
 */

params ["_value"];

if !(_value isEqualType []) exitWith { _value };

switch (_value select 0) do {
    case "A": {
        (_value select 1) apply { [_x] call FUNC(ae3_decode) }
    };
    case "H": {
        private _pairs = (_value select 1) apply { [_x select 0, [_x select 1] call FUNC(ae3_decode)] };
        createHashMapFromArray _pairs
    };
    case "C": {
        compile preprocessFileLineNumbers (_value select 1)
    };
    case "O": {
        private _id = _value select 1;
        if (_id == "") exitWith { objNull };
        private _found = allMissionObjects "All" select { (_x getVariable [QGVAR(id), ""]) == _id };
        if (_found isEqualTo []) then { objNull } else { _found select 0 }
    };
    default {
        ""
    };
};
