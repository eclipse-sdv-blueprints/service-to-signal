/*******************************************************************************
* Copyright (c) 2024 Contributors to the Eclipse Foundation
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Eclipse Public License 2.0 which is available at
* http://www.eclipse.org/legal/epl-2.0
*
* SPDX-License-Identifier: EPL-2.0
*******************************************************************************/

//extern crate prost_build;
extern crate protobuf_codegen;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    protobuf_codegen::Codegen::new()
        .protoc()
        // use vendored protoc instead of relying on user provided protobuf installation
        .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
        .include("proto/")
        .inputs(["proto/uprotocol/uoptions.proto", "proto/uservices_options.proto", "proto/units.proto", "proto/google/rpc/status.proto", "proto/vehicle/body/horn/v1/horn_service.proto", "proto/vehicle/body/horn/v1/horn_topics.proto", ])
        .cargo_out_dir("uservice")
        .run_from_script();
    Ok(())
}