#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

["ace_treatmentSucceded", FUNC(ace_onTreatmentSucceded)] call CBA_fnc_addEventHandler;
