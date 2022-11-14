#include "script_component.hpp"

params ["_name"];

if !(GVAR(loading)) exitWith {
    WARNING_1("attempting to register %1 while not loading", _name);
};

GVAR(loaders) set [_name, false];
INFO_1("Loader %1 registered", _name);
