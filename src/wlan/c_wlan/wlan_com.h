#include <stdbool.h>

//turn on or off a given wireless interface based on a given state.
//state = 1 -> ON, state = 0 -> OFF
//RETURN: zero on success, -1 on error.
int c_toggle_power(char *iface,bool state);

//turn on or off monitor mode of a given wireless interface based on a given state.
//state = 1 -> ON, state = 0 -> OFF
//RETURN: zero on success, -1 on error.
int c_toggle_monitor_mode(char *iface,bool state);


//change the listenning channel of an interface
//RETURN: zero on success, -1 on error.
int c_switch_channel(char *iface,unsigned channel);

//gets the current channel of a given interface
//RETURN: channel on success, -1 on error.
int c_get_channel(char *iface);
