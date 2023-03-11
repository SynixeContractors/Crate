#include "script_component.hpp"

params ["_unit"];

_unit setVariable [QGVAR(unconsciousSince), CBA_missionTime, true];
