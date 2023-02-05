#include "script_component.hpp"

params ["_vehicle", "_type", "_index"];

((getPylonMagazines _vehicle) select (_index - 1)) == _type
