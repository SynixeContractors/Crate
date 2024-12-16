#include "script_component.hpp"

params ["_unit"];

_unit setVariable [QGVAR(unconsciousDelay), CBA_missionTime + 2, true];
