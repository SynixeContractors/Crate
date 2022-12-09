#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

[QGVAR(updatedId), {
    call FUNC(diaryEntry);
    [QGVAR(saveDLC), [player, getDLCs 1]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;

[QGVAR(updatedRoles), {
    call FUNC(diaryEntry);
    if !(GVAR(roles_enabled)) exitWith {};
    call FUNC(setTraits);
}] call CBA_fnc_addEventHandler;

[QGVAR(needsLink), {
    ["Your account was not able to be automatically linked. Ensure that your Discord nickname and Arma 3 name match. You can add `-name=""First Last""` to the Swifty parameters."] spawn BIS_fnc_guiMessage;
}] call CBA_fnc_addEventHandler;
