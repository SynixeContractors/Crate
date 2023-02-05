#include "script_component.hpp"

params ["_vehicle"];

private _state = createHashMap;

// Fuel
_state set ["fuel", fuel _vehicle];
_state set ["ace_fuel", _vehicle getVariable ["ace_fuel_fuel", 0]];

// ACE Cargo
if (missionNamespace getVariable ["ace_cargo", false]) then {
    private _cargo = [];
    {
        _cargo pushBack (if (_x isEqualType "") then {
            [0, _x]
        } else {
            [1, [typeOf _x, [_x] call FUNC(saveState)]]
        });
    } forEach (_vehicle getVariable ["ace_cargo_loaded", []]);
    if (_cargo isNotEqualTo []) then {
        _state set ["cargo", _cargo];
    };
};


// Textures
private _tex = getObjectTextures _vehicle;
if (_tex isNotEqualTo []) then {
    _state set ["tex", _tex];
};

// Damage
private _hits = [];
private _data = (getAllHitPointsDamage _vehicle);
{
    private _damage = _data select 2 select _forEachIndex;
    if (_damage != 0) then {
        _hits pushBack [_x, _damage];
    };
} forEach (_data select 0);
if (_hits isNotEqualTo []) then {
    _state set ["hits", _hits];
};

// Inventory
private _tree = [_vehicle] call FUNC(getInventory);
if (_tree isNotEqualTo [[[[],[]],[],[[],[]]],[]]) then {
    _state set ["inventory", _tree];
};

// Cargo Inventory
private _ammoCargo = getAmmoCargo _vehicle;
if (_ammoCargo isNotEqualTo -1) then {
    _state set ["ammoCargo", _ammoCargo];
};
private _repairCargo = getRepairCargo _vehicle;
if (_repairCargo isNotEqualTo -1) then {
    _state set ["repairCargo", _ammoCargo];
};
private _fuelCargo = getFuelCargo _vehicle;
if (_fuelCargo != -1) then {
    _state set ["fuelCargo", _fuelCargo];
};
private _ammo = magazinesAmmo _vehicle;
if (_ammo isNotEqualTo []) then {
    _state set ["ammo", _ammo];
};

// Turret Ammo
private _turretAmmo = magazinesAllTurrets _vehicle;
if (_turretAmmo isNotEqualTo []) then {
    _state set ["turret", _turretAmmo];
};

_state
