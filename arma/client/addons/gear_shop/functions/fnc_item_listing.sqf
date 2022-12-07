#include "script_component.hpp"

params ["_class"];

private _ret = _class;

// Check for shop listing
private _shopClass = GVAR(items) getOrDefault [_class, []];
if !(_shopClass isEqualTo []) exitWith {
    _class
};

// Handles CBA disposable rockets
private _launcherCheck = _class splitString "_";
if (count _launcherCheck > 0) then {
    if ((tolower (_launcherCheck select (count _launcherCheck - 1))) isEqualTo "loaded") then {
        _launcherCheck deleteAt (count _launcherCheck - 1);
        _launcherCheck = _launcherCheck joinString "_";
        private _shopClass = GVAR(items) getOrDefault [_launcherCheck, []];
        if !(_shopClass isEqualTo []) then {
            _ret = _launcherCheck;
        };
    };
};

// Check for non-pip
private _parents = [configFile >> "CfgWeapons" >> _class, true] call BIS_fnc_returnParents;
if (count _parents > 2) then {
    private _shopClass = GVAR(items) getOrDefault [_parents select 1, []];
    if !(_shopClass isEqualTo []) then {
        _ret = (_parents select 1);
    };
};

// Check for MRT next scope
private _nextClass = configFile >> "CfgWeapons" >> _class >> "MRT_SwitchItemNextClass";
if (isText (_nextClass)) then {
    private _next = getText _nextClass;
    private _shopClass = GVAR(items) getOrDefault [_next, []];
    if !(_shopClass isEqualTo []) then {
        _ret = _next;
    };
};

// Check for ACRE
private _acreCheck = (tolower _class) splitString "_";
if (count _acreCheck == 4) then {
    if (_acreCheck select 0 isEqualTo "acre") then {
        if (_acreCheck select 2 isEqualTo "id") then {
            _ret = toupper format ["acre_%1", _acreCheck select 1];
        };
    };
};

_ret
