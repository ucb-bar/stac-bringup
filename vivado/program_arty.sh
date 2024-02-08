#!/bin/bash
HOME_DIR=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
vivado_lab -mode batch -source $HOME_DIR/program_arty.tcl

