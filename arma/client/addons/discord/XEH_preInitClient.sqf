#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[QGVAR(updatedId), {
    call FUNC(diaryEntry);
}] call CBA_fnc_addEventHandler;

[QGVAR(updatedRoles), {
    call FUNC(diaryEntry);
    call FUNC(setTraits);
}] call CBA_fnc_addEventHandler;

[QGVAR(needsLink), {
    ["Your account was not able to be automatically linked. Ensure that your Discord nickname and Arma 3 name match. You can add `-name=""First Last""` to the Swifty parameters."] spawn BIS_fnc_guiMessage;
}] call CBA_fnc_addEventHandler;
