#include "script_component.hpp"

if !(hasInterface) exitWith {};

["Crate", "Restore AE3 Device", FUNC(ae3_zeusRestore)] call zen_custom_modules_fnc_register;
