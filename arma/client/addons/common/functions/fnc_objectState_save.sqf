#include "script_component.hpp"

params ["_object"];

private _state = createHashMap;

// Engine & Lights
if (_object isKindOf "Car") then {
    _state set ["engine", isEngineOn _object];
    _state set ["light", isLightOn _object];
};
if (_object isKindOf "Tank") then {
    _state set ["engine", isEngineOn _object];
    _state set ["light", isLightOn _object];
};
if (_object isKindOf "Helicopter") then {
    _state set ["engine", isEngineOn _object];
    _state set ["light", isLightOn _object];
    _state set ["collision", isCollisionLightOn _object];
};

// Inflamed
if (inflamed _object) then {
    _state set ["inflamed", true];
};

// Fuel
_state set ["fuel", fuel _object];
_state set ["ace_fuel", _object getVariable ["ace_fuel_fuel", 0]];

// Plate
_state set ["plate", getPlateNumber _object];

// ACE Cargo
if (missionNamespace getVariable ["ace_cargo", false]) then {
    private _cargoSpace = _object getVariable ["ace_cargo_space", -100];
    if (_cargoSpace != -100) then {
        _state set ["cargo_space", _cargoSpace];
    };
    private _cargo = [];
    {
        _cargo pushBack (if (_x isEqualType "") then {
            [0, _x]
        } else {
            [1, [typeOf _x, [_x] call FUNC(objectState_save)]]
        });
    } forEach (_object getVariable ["ace_cargo_loaded", []]);
    if (_cargo isNotEqualTo []) then {
        _state set ["cargo", _cargo];
    };
};

// ACE Captives
if (_object getVariable ["ace_captives_isHandcuffed", false]) then {
    _state set ["handcuffed", true];
};

// Textures
private _tex = getObjectTextures _object;
if (_tex isNotEqualTo []) then {
    _state set ["tex", _tex];
};

// Damage
if !(alive _object) then {
    _state set ["dead", true];
} else {
    private _hits = [];
    private _data = (getAllHitPointsDamage _object);
    {
        private _damage = _data select 2 select _forEachIndex;
        if (_damage != 0) then {
            _hits pushBack [_x, _damage];
        };
    } forEach (_data select 0);
    if (_hits isNotEqualTo []) then {
        _state set ["hits", _hits];
    };
};

// Inventory
private _tree = [_object] call FUNC(getInventory);
if (_tree isNotEqualTo [[[[],[]],[],[[],[]]],[]]) then {
    _state set ["inventory", _tree];
};

// ACE Rearm
if (_object getVariable ["ace_rearm_magazineClass", ""] isNotEqualTo "") then {
    _state set ["ace_mag", _object getVariable ["ace_rearm_magazineClass", ""]];
};

// Cargo Inventory
private _ammoCargo = getAmmoCargo _object;
if (_ammoCargo isNotEqualTo -1) then {
    _state set ["ammoCargo", _ammoCargo];
};
private _repairCargo = getRepairCargo _object;
if (_repairCargo isNotEqualTo -1) then {
    _state set ["repairCargo", _ammoCargo];
};
private _fuelCargo = getFuelCargo _object;
if (_fuelCargo != -1) then {
    _state set ["fuelCargo", _fuelCargo];
};
private _ammo = magazinesAmmo _object;
if (_ammo isNotEqualTo []) then {
    _state set ["ammo", _ammo];
};

// Turret Ammo
private _turretAmmo = magazinesAllTurrets _object;
if (_turretAmmo isNotEqualTo []) then {
    _state set ["turret", _turretAmmo];
};

// Animation Phases
private _phases = [];
{
    private _phase = _object animationSourcePhase _x;
    _phases pushBack [_x, _phase];
} forEach (animationNames _object);
if (_phases isNotEqualTo []) then {
    _state set ["phases", _phases];
};

// Locked
private _locked = locked _object;
if !(_locked in [-1, 0]) then {
    _state set ["locked", _locked];
};

// Terrain Object
if (_object getVariable [QGVAR(terrain), ""] isNotEqualTo "") then {
    _state set ["terrain", _object getVariable [QGVAR(terrain), ""]];
};

_state
