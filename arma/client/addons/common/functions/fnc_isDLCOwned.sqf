#include "script_component.hpp"

params ["_class"];

([_class] call ace_common_fnc_getDLC) params ["", "_id"];

if (_id == 0) exitWith {true};

isDLCAvailable _id
