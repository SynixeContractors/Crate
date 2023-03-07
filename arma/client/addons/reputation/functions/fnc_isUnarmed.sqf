#include "script_component.hpp"

params ["_unit"];

if (primaryWeapon _unit != "") exitWith {false};
if (secondaryWeapon _unit != "") exitWith {false};
if (handgunWeapon _unit != "") exitWith {false};

true
