#include "script_component.hpp"

player setVariable ["ace_medical_medicClass", 0, true];
player setVariable ["ACE_isEngineer", 0, true];
{
    if (_x == "780136967677411389") then {
        player setVariable ["ace_medical_medicClass", 1, true];
    };
    if (_x in ["1038406302935810078", "1038406347114426410", "1038406394925301792"]) then {
        player setVariable ["ACE_isEngineer", true, true];
    };
} forEach (player getVariable [QGVAR(roles), []]);
