#include "script_component.hpp"

params ["_name"];

if !(GVAR(loading)) exitWith {
    WARNING_1("attempting to error %1 while not loading", _name);
};

GVAR(loaders) set [_name, true];
INFO_1("Loader %1 error", _name);

GVAR(blurHandle) ppEffectAdjust [0];
GVAR(blurHandle) ppEffectCommit 0.5;
[{
    GVAR(blurHandle) ppEffectEnable false;
    ppEffectDestroy GVAR(blurHandle);
}, [], 1] call CBA_fnc_waitAndExecute;

[QGVAR(reverting)] call CBA_fnc_localEvent;

player enableSimulation true;
