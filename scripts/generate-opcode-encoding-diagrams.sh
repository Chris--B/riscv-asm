#!/bin/bash

# Make sure you have protocol from here:
#   https://github.com/luismartingarcia/protocol
# Note: We'll reverse the order when copying into source, to match the HW spec.

set -e

protocol --version
echo

cmd="protocol -b 32"

echo "R-type"
$cmd "opcode:7,rd:5,funct3:3,rs1:5,rs2:5,funct7:7"
echo

echo "I-type"
$cmd "opcode:7,rd:5,funct3:3,rs1:5,imm[11;0]:12"
echo

echo "S-type"
$cmd "opcode:7,imm[4;0]:5,funct3:3,rs1:5,rs2:5,imm[11;5]:7"
echo

echo "U-type"
$cmd "opcode:7,rd:5,imm[31;12]:20"
echo

