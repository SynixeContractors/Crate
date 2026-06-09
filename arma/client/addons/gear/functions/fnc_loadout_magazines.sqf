#include "..\script_component.hpp"

private _loadout = getUnitLoadout player;

private _magazines = createHashMap;
private _bulletsPerMagazine = createHashMap;

_fnc_getPerMag = {
    params ["_class"];
    _bulletsPerMagazine getOrDefaultCall [_class, {
        getNumber (configFile >> "CfgMagazines" >> _class >> "count")
    }];
};

_fnc_addBullets = {
    params ["_class", "_count"];
    if (getNumber (configFile >> "CfgMagazines" >> _class >> "ace_disableRepacking") == 1) exitWith {};
    private _needed = ([_class] call _fnc_getPerMag) - _count;
    private _existing = _magazines getOrDefault [_class, 0];
    _magazines set [_class, _existing + _needed];
};

_fnc_doWeapon = {
    params ["_weaponArray"];
    if (_weaponArray isNotEqualTo []) then {
        {
            private _mag = _weaponArray select _x;
            if (_mag isNotEqualTo []) then {
                _mag call _fnc_addBullets;
                _mag set [1, (_mag select 0) call _fnc_getPerMag];
            };
        } forEach [4, 5];
    };
};

_fnc_doContainer = {
    params ["_containerArray"];
    if (_containerArray isEqualTo []) exitWith {};
    {
        if (count _x == 3) then {
            for "_i" from 1 to (_x select 1) do {
                [_x select 0, _x select 2] call _fnc_addBullets;
            };
            _x set [2, (_x select 0) call _fnc_getPerMag];
        };
        if (count _x == 2) then {
            if (typeName (_x select 0) == "ARRAY") then {
                for "_i" from 1 to (_x select 1) do {
                    [_x select 0] call _fnc_doWeapon;
                };
            };
        };
    } forEach (_containerArray select 1);
};

// Primary
[_loadout select 0] call _fnc_doWeapon;

// Secondary
[_loadout select 1] call _fnc_doWeapon;

// Pistol
[_loadout select 2] call _fnc_doWeapon;

// Uniform
[_loadout select 3] call _fnc_doContainer;

// Vest
[_loadout select 4] call _fnc_doContainer;

// Backpack
[_loadout select 5] call _fnc_doContainer;

{
    _magazines set [_x, [_y, [_x] call _fnc_getPerMag]];
} forEach _magazines;

player setUnitLoadout _loadout;

_magazines
