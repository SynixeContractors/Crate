#include "script_component.hpp"

params ["_class"];

private _ret = _class;

// Check for shop listing
private _shopClass = GVAR(shop_items) getOrDefault [_class, []];
if (_shopClass isNotEqualTo []) exitWith {
    _class
};

// Handles CBA disposable rockets
private _launcherCheck = _class splitString "_";
if (count _launcherCheck > 0) then {
    if ((toLower (_launcherCheck select -1)) isEqualTo "loaded") then {
        _launcherCheck deleteAt [-1];
        _launcherCheck = _launcherCheck joinString "_";
        private _shopClass = GVAR(shop_items) getOrDefault [_launcherCheck, []];
        if (_shopClass isNotEqualTo []) then {
            _ret = _launcherCheck;
        };
    };
};

// Check for non-pip
private _parents = [configFile >> "CfgWeapons" >> _class, true] call BIS_fnc_returnParents;
if (count _parents > 2) then {
    private _shopClass = GVAR(shop_items) getOrDefault [_parents select 1, []];
    if (_shopClass isNotEqualTo []) then {
        _ret = (_parents select 1);
    };
};

// Check for MRT next scope
private _nextClass = configFile >> "CfgWeapons" >> _class >> "MRT_SwitchItemNextClass";
if (isText (_nextClass)) then {
    private _next = getText _nextClass;
    private _shopClass = GVAR(shop_items) getOrDefault [_next, []];
    if (_shopClass isNotEqualTo []) then {
        _ret = _next;
    };
};

// Check for ACRE
private _acreCheck = (toLower _class) splitString "_";
if (count _acreCheck == 4) then {
    if (_acreCheck select 0 isEqualTo "acre") then {
        if (_acreCheck select 2 isEqualTo "id") then {
            _ret = toUpper format ["acre_%1", _acreCheck select 1];
        };
    };
};

_ret
