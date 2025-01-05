#include "script_component.hpp"

params ["_class"];

([_class] call ace_common_fnc_getDLC) params ["", "_id"];

// Remove with ACE 3.18.2
if !(_id isEqualType 0) then {
    _id = parseNumber _id;
};

if (_id isEqualTo 0) exitWith { true };

isDLCAvailable _id
