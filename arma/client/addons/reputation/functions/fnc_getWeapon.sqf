params ["_unit"];

if (isNull objectParent _unit) exitWith {
    currentWeapon _unit
};

private _role = assignedVehicleRole _unit;

if (_role#0 != "turret") exitWith {
    typeOf (vehicle _unit)
};

((vehicle _unit) weaponsTurret [0]) params [["_weapon", typeOf (vehicle _unit)]];

_weapon
