#ifndef MIXER_NODE_HPP
#define MIXER_NODE_HPP

#include "genesis.hpp"

int create_mixer_descriptor(GenesisPipeline *pipeline, int input_port_count,
        GenesisNodeDescriptor **out);

#endif

