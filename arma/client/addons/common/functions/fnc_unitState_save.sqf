#include "script_component.hpp"

params ["_unit"];

private _state = createHashMap;

_state set ["name", name _unit];
_state set ["loadout", [_unit] call CBA_fnc_getLoadout];

if (face _unit != "Default") then {
    _state set ["face", face _unit];
};
if (speaker _unit != "") then {
    _state set ["speaker", speaker _unit];
};
if (rank _unit != "PRIVATE") then {
    _state set ["rank", rank _unit];
};
if (pitch _unit != 1) then {
    _state set ["pitch", pitch _unit];
};
if !(alive _unit) then {
    _state set ["alive", false];
};
if (_unit isFlashlightOn (currentWeapon _unit)) then {
    _state set ["flashlight", true];
};
if (_unit isIRLaserOn (currentWeapon _unit)) then {
    _state set ["irlaser", true];
};
if (primaryWeapon _unit != currentWeapon _unit) then {
    _state set ["weapon", currentWeapon _unit];
};
if (unitPos _unit != "Auto") then {
    _state set ["unitPos", unitPos _unit];
};
if (unitCombatMode _unit != "YELLOW") then {
    _state set ["combat", unitCombatMode _unit];
};
if (behaviour _unit != "NORMAL") then {
    _state set ["behaviour", behaviour _unit];
};
if !(isNull objectParent _unit) then {
    private _vehicle = objectParent _unit;
    _state set ["vehicle", [
        _vehicle getVariable [QEGVAR(campaigns,id), ""],
        call {
            if (driver _vehicle isEqualTo _unit) exitWith { "driver" };
            if (gunner _vehicle isEqualTo _unit) exitWith { "gunner" };
            if (commander _vehicle isEqualTo _unit) exitWith { "commander" };
            _vehicle getCargoIndex _unit
        }
    ]];
};
private _disableAI = [];
{
    if !(_unit checkAIFeature _x) then {
        _disableAI pushBack _x;
    };
} forEach ["ALL","AUTOTARGET","MOVE","TARGET","TEAMSWITCH","WEAPONAIM","ANIM","FSM","AIMINGERROR","SUPPRESSION","CHECKVISIBLE","AUTOCOMBAT","COVER","PATH","MINEDETECTION","LIGHTS","NVG","RADIOPROTOCOL","FIREWEAPON"];
if (count _disableAI > 0) then {
    _state set ["disableAI", _disableAI];
};

if (missionNamespace getVariable ["ace_main", false]) then {
    if (_unit getVariable ["ace_captives_isSurrendering", false]) then {
        _state set ["ace_surrender", true];
    };
    if (_unit getVariable ["ace_captives_isHandcuffed", false]) then {
        _state set ["ace_handcuffed", true];
    };
};

_state
