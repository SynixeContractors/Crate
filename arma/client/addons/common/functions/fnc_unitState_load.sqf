#include "script_component.hpp"

params ["_unit", "_state"];

{
    switch (_x) do {
        case "name": {
            [_unit, _y] remoteExec ["setName", 0, _unit];
            [_unit] call ace_common_fnc_setName;
        };
        case "loadout": {
            [_unit, _y] call CBA_fnc_setLoadout;
        };
        case "face": {
            [_unit, _y] remoteExec ["setFace", 0, _unit];
        };
        case "speaker": {
            [_unit, _y] remoteExec ["setSpeaker", 0, _unit];
        };
        case "rank": {
            _unit setRank _y;
        };
        case "pitch": {
            [_unit, _y] remoteExec ["setPitch", 0, _unit];
        };
        case "alive": {
            if !(_y) then {
                _unit setDamage [1, false];
            };
        };
        case "flashlight": {
            _unit enableGunLights "ForceOn";
        };
        case "irlaser": {
            _unit enableIRLasers _y;
        };
        // case "weapon"
        case "unitPos": {
            _unit setUnitPos _y;
        };
        case "combat": {
            _unit setUnitCombatMode _y;
        };
        case "behaviour": {
            _unit setCombatBehaviour _y;
        };
        case "vehicle": {
            _y params ["_vid", "_seat"];
            private _objects = allMissionObjects "All";
            private _vehicle = _objects select (_objects findIf { (_x getVariable [QEGVAR(campaigns,id), "-"]) isEqualTo _vid });
            switch (_seat) do {
                case "driver": {
                    _unit moveInDriver _vehicle;
                };
                case "gunner": {
                    _unit moveInGunner _vehicle;
                };
                case "commander": {
                    _unit moveInCommander _vehicle;
                };
                default {
                    _unit moveInCargo [_vehicle, _seat];
                };
            };
        };
        case "stopped": {
            doStop _unit;
        };
        case "disableAI": {
            {
                _unit disableAI _x;
            } forEach _y;
        };
        case "skills": {
            {
                _unit setSkill [_x#0, _x#1];
            } forEach _y;
        };
        case "ace_surrender": {
            [_unit, true] call ACE_captives_fnc_setSurrendered;
        };
        case "ace_handcuffed": {
            [_unit, true] call ACE_captives_fnc_setHandcuffed;
        };
    };
} forEach _state;
