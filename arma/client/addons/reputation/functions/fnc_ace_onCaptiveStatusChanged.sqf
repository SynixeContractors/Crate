#include "script_component.hpp"

params ["_unit", "_state", "_event"];

if (_event == "SetSurrendered") then {
    if (_state) then {
        _unit setVariable [QGVAR(surrenderDelay), CBA_missionTime + 2];
    };
};
