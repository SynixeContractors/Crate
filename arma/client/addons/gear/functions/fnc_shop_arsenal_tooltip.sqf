#include "script_component.hpp"

params ["_raw_class"];

private _class = [_raw_class] call FUNC(shop_item_listing);

private _color = COLOR_WHITE;

private _price = [_class, false] call FUNC(shop_item_price);
private _personal = _price select 0;
private _company = _price select 1;
private _personal_base = _price select 2;
private _company_base = _price select 3;

private _cost = format ["Personal: %1", _personal];

if (_personal > _personal_base) then {
    _color = COLOR_INCREASE;
    _cost = format ["%1\nRegular: %2", _cost, _personal_base];
};
if (_personal < _personal_base) then {
    _color = COLOR_SALE;
    _cost = format ["%1\nRegular: %2", _cost, _personal_base];
};

if (_company != 0) then {
    _cost = format ["%1\nCompany: %2", _cost, _company];
    if (_company > _company_base) then {
        _cost = format ["%1\nRegular: %2", _cost, _company_base];
    };
    if (_company < _company_base) then {
        _cost = format ["%1\nRegular: %2", _cost, _company_base];
    };
};

private _equipped = _items getOrDefault [_raw_class, 0];
private _tooltip = if (GVAR(readOnly)) then {
    format ["%1\nEquipped: %2\n%3", _raw_class, _equipped, _cost]
} else {
    private _owned = [_raw_class] call FUNC(shop_item_owned);
    if (_owned > 0) then {
        _color = COLOR_OWNED;
    };
    format ["%1\nOwned: %2\nEquipped: %3\n%4", _raw_class, _owned, _equipped, _cost]
};

[_tooltip, _color]
