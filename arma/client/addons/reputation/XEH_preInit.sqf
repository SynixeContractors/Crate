#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

["ace_treatmentSucceded", { call FUNC(ace_onTreatmentSucceded) }] call CBA_fnc_addEventHandler;
["ace_medical_knockOut", { call FUNC(ace_onKnockOut) }] call CBA_fnc_addEventHandler;
["ace_hitreactions_dropWeapon", { call FUNC(ace_onDropWeapon) }] call CBA_fnc_addEventHandler;
["ace_captiveStatusChanged", { call FUNC(ace_onCaptiveStatusChanged)}] call CBA_fnc_addEventHandler;
