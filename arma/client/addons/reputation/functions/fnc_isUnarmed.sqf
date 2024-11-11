#include "script_component.hpp"

params ["_unit"];

if (!isNull objectParent _unit) exitWith {false};
if (primaryWeapon _unit != "") exitWith {false};
if (secondaryWeapon _unit != "") exitWith {false};
if (handgunWeapon _unit != "") exitWith {false};

true
